<details>
<summary>XSD contract: <code>DataComponentValueSetType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="DataComponentValueSetType">
		<xs:annotation>
			<xs:documentation>DataComponentValueSetType defines the structure for providing values for a data attributes, measures, or metadata attributes. If no values are provided, the component is implied to include/excluded from the region in which it is defined, with no regard to the value of the component. Note that for metadata attributes which occur within other metadata attributes, a nested identifier can be provided. For example, a value of CONTACT.ADDRESS.STREET refers to the metadata attribute with the identifier STREET which exists in the ADDRESS metadata attribute in the CONTACT metadata attribute, which is defined at the root of the report structure.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="MemberSelectionType">
				<xs:choice minOccurs="0">
					<xs:element name="Value" type="DataComponentValueType" maxOccurs="unbounded"/>
					<xs:element name="TimeRange" type="TimeRangeValueType"/>
				</xs:choice>
				<xs:attribute name="id" type="common:NestedNCNameIDType" use="required"/>
				<xs:attribute name="validFrom" type="common:StandardTimePeriodType" use="prohibited"/>
				<xs:attribute name="validTo" type="common:StandardTimePeriodType" use="prohibited"/>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
