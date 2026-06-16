<details>
<summary>XSD contract: <code>DataType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:simpleType name="DataType">
		<xs:annotation>
			<xs:documentation>DataTypeType provides an enumerated list of the types of data formats allowed as the for the representation of an object.</xs:documentation>
		</xs:annotation>
		<xs:restriction base="xs:NMTOKEN">
			<xs:enumeration value="String">
				<xs:annotation>
					<xs:documentation>A string datatype corresponding to W3C XML Schema's xs:string datatype.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="Alpha">
				<xs:annotation>
					<xs:documentation>A string datatype which only allows for the simple alphabetic character set of A-Z, a-z.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="AlphaNumeric">
				<xs:annotation>
					<xs:documentation>A string datatype which only allows for the simple alphabetic character set of A-Z, a-z plus the simple numeric character set of 0-9.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="Numeric">
				<xs:annotation>
					<xs:documentation>A string datatype which only allows for the simple numeric character set of 0-9. This format is not treated as an integer, and therefore can having leading zeros.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="BigInteger">
				<xs:annotation>
					<xs:documentation>An integer datatype corresponding to W3C XML Schema's xs:integer datatype.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="Integer">
				<xs:annotation>
					<xs:documentation>An integer datatype corresponding to W3C XML Schema's xs:int datatype.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="Long">
				<xs:annotation>
					<xs:documentation>A numeric datatype corresponding to W3C XML Schema's xs:long datatype.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="Short">
				<xs:annotation>
					<xs:documentation>A numeric datatype corresponding to W3C XML Schema's xs:short datatype.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="Decimal">
				<xs:annotation>
					<xs:documentation>A numeric datatype corresponding to W3C XML Schema's xs:decimal datatype.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="Float">
				<xs:annotation>
					<xs:documentation>A numeric datatype corresponding to W3C XML Schema's xs:float datatype.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="Double">
				<xs:annotation>
					<xs:documentation>A numeric datatype corresponding to W3C XML Schema's xs:double datatype.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="Boolean">
				<xs:annotation>
					<xs:documentation>A datatype corresponding to W3C XML Schema's xs:boolean datatype.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="URI">
				<xs:annotation>
					<xs:documentation>A datatype corresponding to W3C XML Schema's xs:anyURI datatype.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="Count">
				<xs:annotation>
					<xs:documentation>A simple incrementing Integer type. The isSequence facet must be set to true, and the interval facet must be set to "1".</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="InclusiveValueRange">
				<xs:annotation>
					<xs:documentation>This value indicates that the startValue and endValue attributes provide the inclusive boundaries of a numeric range of type xs:decimal.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="ExclusiveValueRange">
				<xs:annotation>
					<xs:documentation>This value indicates that the startValue and endValue attributes provide the exclusive boundaries of a numeric range, of type xs:decimal.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="Incremental">
				<xs:annotation>
					<xs:documentation>This value indicates that the value increments according to the value provided in the interval facet, and has a true value for the isSequence facet.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="ObservationalTimePeriod">
				<xs:annotation>
					<xs:documentation>Observational time periods are the superset of all time periods in SDMX. It is the union of the standard time periods (i.e. Gregorian time periods, the reporting time periods, and date time) and a time range.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="StandardTimePeriod">
				<xs:annotation>
					<xs:documentation>Standard time periods is a superset of distinct time period in SDMX. It is the union of the basic time periods (i.e. the Gregorian time periods and date time) and the reporting time periods.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="BasicTimePeriod">
				<xs:annotation>
					<xs:documentation>BasicTimePeriod time periods is a superset of the Gregorian time periods and a date time.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="GregorianTimePeriod">
				<xs:annotation>
					<xs:documentation>Gregorian time periods correspond to calendar periods and are represented in ISO-8601 formats. This is the union of the year, year month, and date formats.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="GregorianYear">
				<xs:annotation>
					<xs:documentation>A Gregorian time period corresponding to W3C XML Schema's xs:gYear datatype, which is based on ISO-8601.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="GregorianYearMonth">
				<xs:annotation>
					<xs:documentation>A time datatype corresponding to W3C XML Schema's xs:gYearMonth datatype, which is based on ISO-8601.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="GregorianDay">
				<xs:annotation>
					<xs:documentation>A time datatype corresponding to W3C XML Schema's xs:date datatype, which is based on ISO-8601.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="ReportingTimePeriod">
				<xs:annotation>
					<xs:documentation>Reporting time periods represent periods of a standard length within a reporting year, where to start of the year (defined as a month and day) must be defined elsewhere or it is assumed to be January 1. This is the union of the reporting year, semester, trimester, quarter, month, week, and day.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="ReportingYear">
				<xs:annotation>
					<xs:documentation>A reporting year represents a period of 1 year (P1Y) from the start date of the reporting year. This is expressed as using the SDMX specific ReportingYearType.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="ReportingSemester">
				<xs:annotation>
					<xs:documentation>A reporting semester represents a period of 6 months (P6M) from the start date of the reporting year. This is expressed as using the SDMX specific ReportingSemesterType.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="ReportingTrimester">
				<xs:annotation>
					<xs:documentation>A reporting trimester represents a period of 4 months (P4M) from the start date of the reporting year. This is expressed as using the SDMX specific ReportingTrimesterType.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="ReportingQuarter">
				<xs:annotation>
					<xs:documentation>A reporting quarter represents a period of 3 months (P3M) from the start date of the reporting year. This is expressed as using the SDMX specific ReportingQuarterType.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="ReportingMonth">
				<xs:annotation>
					<xs:documentation>A reporting month represents a period of 1 month (P1M) from the start date of the reporting year. This is expressed as using the SDMX specific ReportingMonthType.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="ReportingWeek">
				<xs:annotation>
					<xs:documentation>A reporting week represents a period of 7 days (P7D) from the start date of the reporting year. This is expressed as using the SDMX specific ReportingWeekType.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="ReportingDay">
				<xs:annotation>
					<xs:documentation>A reporting day represents a period of 1 day (P1D) from the start date of the reporting year. This is expressed as using the SDMX specific ReportingDayType.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="DateTime">
				<xs:annotation>
					<xs:documentation>A time datatype corresponding to W3C XML Schema's xs:dateTime datatype.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="TimeRange">
				<xs:annotation>
					<xs:documentation>TimeRange defines a time period by providing a distinct start (date or date time) and a duration.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="Month">
				<xs:annotation>
					<xs:documentation>A time datatype corresponding to W3C XML Schema's xs:gMonth datatype.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="MonthDay">
				<xs:annotation>
					<xs:documentation>A time datatype corresponding to W3C XML Schema's xs:gMonthDay datatype.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="Day">
				<xs:annotation>
					<xs:documentation>A time datatype corresponding to W3C XML Schema's xs:gDay datatype.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="Time">
				<xs:annotation>
					<xs:documentation>A time datatype corresponding to W3C XML Schema's xs:time datatype.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="Duration">
				<xs:annotation>
					<xs:documentation>A time datatype corresponding to W3C XML Schema's xs:duration datatype.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="GeospatialInformation">
				<xs:annotation>
					<xs:documentation>A string used to describe geographical features like points (e.g., locations of places, landmarks, buildings, etc.), lines (e.g., rivers, roads, streets, etc.), or areas (e.g., geographical regions, countries, islands, land lots, etc.). A mix of different features is possible too, e.g., combining polygons and geographical points to describe a country and the location of its capital.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="XHTML">
				<xs:annotation>
					<xs:documentation>This value indicates that the content of the component can contain XHTML markup.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="KeyValues">
				<xs:annotation>
					<xs:documentation>This value indicates that the content of the component will be data key (a set of dimension references and values for the dimensions).</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="IdentifiableReference">
				<xs:annotation>
					<xs:documentation>This value indicates that the content of the component will be complete reference (either URN or full set of reference fields) to an Identifiable object in the SDMX Information Model.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
			<xs:enumeration value="DataSetReference">
				<xs:annotation>
					<xs:documentation>This value indicates that the content of the component will be reference to a data provider, which is actually a formal reference to a data provider and a data set identifier value.</xs:documentation>
				</xs:annotation>
			</xs:enumeration>
		</xs:restriction>
	</xs:simpleType>
```

</details>
