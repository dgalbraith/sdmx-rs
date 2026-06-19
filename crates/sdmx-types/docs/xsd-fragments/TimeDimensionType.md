<details>
<summary>XSD contract: <code>TimeDimensionType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="TimeDimensionType">
		<xs:annotation>
			<xs:documentation>TimeDimensionType describes the structure of a time dimension. The time dimension takes its semantic from its concept identity (usually the TIME_PERIOD concept), yet is always has a fixed identifier (TIME_PERIOD). The time dimension always has a fixed text format, which specifies that its format is always the in the value set of the observational time period (see common:ObservationalTimePeriodType). It is possible that the format may be a sub-set of the observational time period value set. For example, it is possible to state that the representation might always be a calendar year. See the enumerations of the textType attribute in the LocalRepresentation/TextFormat for more details of the possible sub-sets. It is also possible to facet this representation with start and end dates. The purpose of such facts is to restrict the value of the time dimension to occur within the specified range. If the time dimension is expected to allow for the standard reporting periods (see common:ReportingTimePeriodType) to be used, then it is strongly recommended that the reporting year start day attribute also be included in the data structure definition. When the reporting year start day attribute is used, any standard reporting period values will be assumed to be based on the start day contained in this attribute. If the reporting year start day attribute is not included and standard reporting periods are used, these values will be assumed to be based on a reporting year which begins January 1.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="BaseDimensionType">
				<xs:sequence>
					<xs:element ref="common:Annotations" minOccurs="0"/>
					<xs:element ref="common:Link" minOccurs="0" maxOccurs="unbounded"/>
					<xs:element name="ConceptIdentity" type="common:ConceptReferenceType"/>
					<xs:element name="LocalRepresentation" type="TimeDimensionRepresentationType"/>
				</xs:sequence>
				<xs:attribute name="urn" type="common:TimeDimensionUrnType" use="optional"/>
				<xs:attribute name="id" type="common:NCNameIDType" use="optional" fixed="TIME_PERIOD"/>
				<xs:attribute name="position" type="xs:int" use="prohibited"/>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
